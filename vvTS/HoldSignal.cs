using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000BB RID: 187
	[HandlerCategory("vvTrade"), HandlerName("Удерживать сигнал N баров")]
	public class HoldSignal : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060006AC RID: 1708 RVA: 0x0001E50D File Offset: 0x0001C70D
		public IList<double> Execute(IList<double> src)
		{
			return HoldSignal.GenHoldSignal(src, this.Bars, this.Context);
		}

		// Token: 0x060006AB RID: 1707 RVA: 0x0001E494 File Offset: 0x0001C694
		public static IList<double> GenHoldSignal(IList<double> signals, int _holdbars, IContext ctx)
		{
			int count = signals.Count;
			double[] array = new double[count];
			int num = 0;
			for (int i = 0; i < count; i++)
			{
				if (i >= _holdbars)
				{
					if (num != 0 && num < _holdbars)
					{
						num++;
						array[i] = 1.0;
					}
					if (signals[i] > 0.0)
					{
						array[i] = 1.0;
						num = 1;
					}
					if (num > _holdbars)
					{
						num = 0;
					}
				}
				else
				{
					array[i] = 0.0;
				}
			}
			return array;
		}

		// Token: 0x1700024D RID: 589
		[HandlerParameter(true, "5", Min = "1", Max = "30", Step = "1")]
		public int Bars
		{
			// Token: 0x060006A9 RID: 1705 RVA: 0x0001E481 File Offset: 0x0001C681
			get;
			// Token: 0x060006AA RID: 1706 RVA: 0x0001E489 File Offset: 0x0001C689
			set;
		}

		// Token: 0x1700024E RID: 590
		public IContext Context
		{
			// Token: 0x060006AD RID: 1709 RVA: 0x0001E521 File Offset: 0x0001C721
			get;
			// Token: 0x060006AE RID: 1710 RVA: 0x0001E529 File Offset: 0x0001C729
			set;
		}
	}
}
