using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200013F RID: 319
	[HandlerCategory("vvRSI"), HandlerDecimals(2), HandlerName("VininI_RSI_IFT")]
	public class VininI_RSI_FO : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060009CF RID: 2511 RVA: 0x00028CA5 File Offset: 0x00026EA5
		public IList<double> Execute(IList<double> src)
		{
			return VininI_RSI_FO.GenRSI_FO(src, this.Context, this.RSI_Period, this.MA_Period);
		}

		// Token: 0x060009CE RID: 2510 RVA: 0x00028B60 File Offset: 0x00026D60
		public static IList<double> GenRSI_FO(IList<double> src, IContext ctx, int _RSI_Period, int _MA_Period)
		{
			Math.Max(_RSI_Period, _MA_Period);
			double[] array = new double[src.Count];
			double[] array2 = new double[src.Count];
			IList<double> data = ctx.GetData("rsi", new string[]
			{
				_RSI_Period.ToString(),
				src.GetHashCode().ToString()
			}, () => Series.RSI(src, _RSI_Period));
			for (int i = 0; i < src.Count; i++)
			{
				array2[i] = 0.1 * (data[i] - 50.0);
			}
			IList<double> list = EMA.EMA_TSLab(array2, _MA_Period);
			for (int j = 2; j < src.Count; j++)
			{
				array[j] = (Math.Exp(2.0 * list[j]) - 1.0) / (Math.Exp(2.0 * list[j]) + 1.0);
			}
			return array;
		}

		// Token: 0x17000336 RID: 822
		public IContext Context
		{
			// Token: 0x060009D0 RID: 2512 RVA: 0x00028CBF File Offset: 0x00026EBF
			get;
			// Token: 0x060009D1 RID: 2513 RVA: 0x00028CC7 File Offset: 0x00026EC7
			set;
		}

		// Token: 0x17000335 RID: 821
		[HandlerParameter(true, "9", Min = "3", Max = "50", Step = "1")]
		public int MA_Period
		{
			// Token: 0x060009CC RID: 2508 RVA: 0x00028B31 File Offset: 0x00026D31
			get;
			// Token: 0x060009CD RID: 2509 RVA: 0x00028B39 File Offset: 0x00026D39
			set;
		}

		// Token: 0x17000334 RID: 820
		[HandlerParameter(true, "5", Min = "3", Max = "25", Step = "1")]
		public int RSI_Period
		{
			// Token: 0x060009CA RID: 2506 RVA: 0x00028B20 File Offset: 0x00026D20
			get;
			// Token: 0x060009CB RID: 2507 RVA: 0x00028B28 File Offset: 0x00026D28
			set;
		}
	}
}
