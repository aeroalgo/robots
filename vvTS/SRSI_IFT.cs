using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200013A RID: 314
	[HandlerCategory("vvRSI"), HandlerName("SmoothedRSI IFT")]
	public class SRSI_IFT : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x0600097D RID: 2429 RVA: 0x00027C89 File Offset: 0x00025E89
		public IList<double> Execute(IList<double> src)
		{
			return SRSI_IFT.GenSRSI(src, this.Context, this.RSI_Period, this.MA_Period);
		}

		// Token: 0x0600097C RID: 2428 RVA: 0x00027B44 File Offset: 0x00025D44
		public static IList<double> GenSRSI(IList<double> src, IContext ctx, int _RSI_Period, int _MA_Period)
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
			IList<double> list = LWMA.GenWMA(array2, _MA_Period);
			for (int j = 2; j < src.Count; j++)
			{
				array[j] = (Math.Exp(2.0 * list[j]) - 1.0) / (Math.Exp(2.0 * list[j]) + 1.0);
			}
			return array;
		}

		// Token: 0x17000315 RID: 789
		public IContext Context
		{
			// Token: 0x0600097E RID: 2430 RVA: 0x00027CA3 File Offset: 0x00025EA3
			get;
			// Token: 0x0600097F RID: 2431 RVA: 0x00027CAB File Offset: 0x00025EAB
			set;
		}

		// Token: 0x17000314 RID: 788
		[HandlerParameter(true, "9", Min = "3", Max = "50", Step = "1")]
		public int MA_Period
		{
			// Token: 0x0600097A RID: 2426 RVA: 0x00027B18 File Offset: 0x00025D18
			get;
			// Token: 0x0600097B RID: 2427 RVA: 0x00027B20 File Offset: 0x00025D20
			set;
		}

		// Token: 0x17000313 RID: 787
		[HandlerParameter(true, "5", Min = "3", Max = "25", Step = "1")]
		public int RSI_Period
		{
			// Token: 0x06000978 RID: 2424 RVA: 0x00027B07 File Offset: 0x00025D07
			get;
			// Token: 0x06000979 RID: 2425 RVA: 0x00027B0F File Offset: 0x00025D0F
			set;
		}
	}
}
