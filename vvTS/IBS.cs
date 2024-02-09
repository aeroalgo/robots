using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000030 RID: 48
	[HandlerCategory("vvIndicators"), HandlerName("Internal Bar Strength")]
	public class IBS : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060001B4 RID: 436 RVA: 0x00008484 File Offset: 0x00006684
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("IBS", new string[]
			{
				this.Length.ToString(),
				sec.get_CacheName()
			}, () => IBS.GenIBS(sec, this.Length, this.Context));
		}

		// Token: 0x060001B3 RID: 435 RVA: 0x00008374 File Offset: 0x00006574
		public static IList<double> GenIBS(ISecurity sec, int _length, IContext _ctx)
		{
			int count = sec.get_Bars().Count;
			IList<double> highPrices = sec.get_HighPrices();
			IList<double> lowPrices = sec.get_LowPrices();
			IList<double> closePrices = sec.get_ClosePrices();
			double[] array = new double[count];
			double[] array2 = new double[count];
			for (int i = 0; i < count; i++)
			{
				double num = highPrices[i] - lowPrices[i];
				if (num > 0.0)
				{
					array2[i] = (closePrices[i] - lowPrices[i]) / num;
				}
				else
				{
					array2[i] = 0.0;
				}
			}
			for (int j = 0; j < count; j++)
			{
				array[j] = vvSeries.iMA(array2, array, 0, _length, j, 0.0, 0.0) * 100.0;
			}
			return array;
		}

		// Token: 0x17000092 RID: 146
		public IContext Context
		{
			// Token: 0x060001B5 RID: 437 RVA: 0x000084E8 File Offset: 0x000066E8
			get;
			// Token: 0x060001B6 RID: 438 RVA: 0x000084F0 File Offset: 0x000066F0
			set;
		}

		// Token: 0x17000091 RID: 145
		[HandlerParameter(true, "5", Min = "3", Max = "10", Step = "1")]
		public int Length
		{
			// Token: 0x060001B1 RID: 433 RVA: 0x00008362 File Offset: 0x00006562
			get;
			// Token: 0x060001B2 RID: 434 RVA: 0x0000836A File Offset: 0x0000656A
			set;
		}
	}
}
