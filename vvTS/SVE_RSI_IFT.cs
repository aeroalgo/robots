using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200013B RID: 315
	[HandlerCategory("vvRSI"), HandlerName("SVE RSI IFT")]
	public class SVE_RSI_IFT : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000986 RID: 2438 RVA: 0x00027F41 File Offset: 0x00026141
		public IList<double> Execute(IList<double> src)
		{
			return SVE_RSI_IFT.GenSRSI(src, this.RSI_Period, this.MA_Period);
		}

		// Token: 0x06000985 RID: 2437 RVA: 0x00027CE0 File Offset: 0x00025EE0
		public static IList<double> GenSRSI(IList<double> src, int _RSI_Period, int _MA_Period)
		{
			Math.Max(_RSI_Period, _MA_Period);
			double[] array = new double[src.Count];
			double[] array2 = new double[src.Count];
			IList<double> list = LWMA.GenWMA(src, 2);
			IList<double> list2 = LWMA.GenWMA(list, 2);
			IList<double> list3 = LWMA.GenWMA(list2, 2);
			IList<double> list4 = LWMA.GenWMA(list3, 2);
			IList<double> list5 = LWMA.GenWMA(list4, 2);
			IList<double> list6 = LWMA.GenWMA(list5, 2);
			IList<double> list7 = LWMA.GenWMA(list6, 2);
			IList<double> list8 = LWMA.GenWMA(list7, 2);
			IList<double> list9 = LWMA.GenWMA(list8, 2);
			IList<double> list10 = LWMA.GenWMA(list9, 2);
			for (int i = 0; i < src.Count; i++)
			{
				double num = 5.0 * list[i] + 4.0 * list2[i] + 3.0 * list3[i] + 2.0 * list4[i] + list5[i] + list6[i] + list7[i] + list8[i] + list9[i] + list10[i];
				num /= 20.0;
				array2[i] = num;
			}
			IList<double> list11 = Series.RSI(array2, _RSI_Period);
			double[] array3 = new double[src.Count];
			for (int j = 0; j < src.Count; j++)
			{
				array3[j] = 0.1 * (list11[j] - 50.0);
			}
			IList<double> list12 = EMA.EMA_TSLab(array3, _MA_Period);
			IList<double> list13 = EMA.EMA_TSLab(list12, _MA_Period);
			double[] array4 = new double[src.Count];
			for (int k = 0; k < src.Count; k++)
			{
				array4[k] = list12[k] + (list12[k] - list13[k]);
			}
			for (int l = 2; l < src.Count; l++)
			{
				array[l] = ((Math.Exp(2.0 * array4[l]) - 1.0) / (Math.Exp(2.0 * array4[l]) + 1.0) + 1.0) * 50.0;
			}
			return array;
		}

		// Token: 0x17000318 RID: 792
		public IContext Context
		{
			// Token: 0x06000987 RID: 2439 RVA: 0x00027F55 File Offset: 0x00026155
			get;
			// Token: 0x06000988 RID: 2440 RVA: 0x00027F5D File Offset: 0x0002615D
			set;
		}

		// Token: 0x17000317 RID: 791
		[HandlerParameter(true, "4", Min = "3", Max = "50", Step = "1")]
		public int MA_Period
		{
			// Token: 0x06000983 RID: 2435 RVA: 0x00027CCD File Offset: 0x00025ECD
			get;
			// Token: 0x06000984 RID: 2436 RVA: 0x00027CD5 File Offset: 0x00025ED5
			set;
		}

		// Token: 0x17000316 RID: 790
		[HandlerParameter(true, "4", Min = "3", Max = "25", Step = "1")]
		public int RSI_Period
		{
			// Token: 0x06000981 RID: 2433 RVA: 0x00027CBC File Offset: 0x00025EBC
			get;
			// Token: 0x06000982 RID: 2434 RVA: 0x00027CC4 File Offset: 0x00025EC4
			set;
		}
	}
}
