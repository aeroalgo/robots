using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000191 RID: 401
	[HandlerCategory("vvAverages"), HandlerName("NRMA")]
	public class NRMA : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000CB3 RID: 3251 RVA: 0x000371EC File Offset: 0x000353EC
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("nrma", new string[]
			{
				this.K.ToString(),
				this.Sharp.ToString(),
				this.MaMinPeriod.ToString(),
				this.Smooth.ToString(),
				this.ShowNRTR.ToString(),
				sec.get_CacheName()
			}, () => NRMA.GenNRMA(sec, this.K, this.Sharp, this.MaMinPeriod, this.Smooth, this.ShowNRTR));
		}

		// Token: 0x06000CB2 RID: 3250 RVA: 0x00036E28 File Offset: 0x00035028
		public static IList<double> GenNRMA(ISecurity sec, double _K, int _Sharp, int _MaMinPeriod, int _Smooth, bool _ShowNRTR)
		{
			int count = sec.get_Bars().Count;
			IList<double> closePrices = sec.get_ClosePrices();
			IList<double> arg_19_0 = sec.get_OpenPrices();
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double[] array4 = new double[count];
			double[] array5 = new double[count];
			double[] array6 = new double[count];
			double[] array7 = new double[count];
			double num = 2.0 / (1.0 + (double)_MaMinPeriod);
			if (sec.get_ClosePrices()[0] > sec.get_OpenPrices()[0])
			{
				array[0] = 1.0;
				array2[0] = closePrices[0] * (1.0 - _K * 0.01);
				array3[0] = 0.0;
				array4[0] = array2[0];
			}
			else
			{
				array[0] = -1.0;
				array3[0] = closePrices[0] * (1.0 + _K * 0.01);
				array2[0] = 0.0;
				array4[0] = array3[0];
			}
			for (int i = 0; i < 100; i++)
			{
				array5[i] = closePrices[i];
			}
			for (int j = 1; j < count; j++)
			{
				if (array[j - 1] > 0.0)
				{
					if (closePrices[j] < array4[j - 1])
					{
						array[j] = -1.0;
						array3[j] = closePrices[j] * (1.0 + _K * 0.01);
						array2[j] = 0.0;
						array4[j] = array3[j];
					}
					else
					{
						array[j] = 1.0;
						array2[j] = closePrices[j] * (1.0 - _K * 0.01);
						array3[j] = 0.0;
						if (array2[j] > array4[j - 1])
						{
							array4[j] = array2[j];
						}
						else
						{
							array4[j] = array4[j - 1];
						}
					}
				}
				else if (closePrices[j] > array4[j - 1])
				{
					array[j] = 1.0;
					array2[j] = closePrices[j] * (1.0 - _K * 0.01);
					array3[j] = 0.0;
					array4[j] = array2[j];
				}
				else
				{
					array[j] = -1.0;
					array3[j] = closePrices[j] * (1.0 + _K * 0.01);
					array2[j] = 0.0;
					if (array3[j] < array4[j - 1])
					{
						array4[j] = array3[j];
					}
					else
					{
						array4[j] = array4[j - 1];
					}
				}
				array6[j] = 100.0 * Math.Abs(closePrices[j] - array4[j]) / closePrices[j] / _K;
			}
			array6[0] = array6[1];
			for (int k = 1; k < count; k++)
			{
				array7[k] = Math.Pow(array6[k], (double)_Sharp);
				array5[k] = array5[k - 1] + array7[k] * num * (closePrices[k] - array5[k - 1]);
			}
			IList<double> result = SMA.GenSMA(array5, _Smooth);
			if (!_ShowNRTR)
			{
				return result;
			}
			return array4;
		}

		// Token: 0x17000427 RID: 1063
		public IContext Context
		{
			// Token: 0x06000CB4 RID: 3252 RVA: 0x00037297 File Offset: 0x00035497
			get;
			// Token: 0x06000CB5 RID: 3253 RVA: 0x0003729F File Offset: 0x0003549F
			set;
		}

		// Token: 0x17000423 RID: 1059
		[HandlerParameter(true, "0.5", Min = "0.1", Max = "10", Step = "0.1")]
		public double K
		{
			// Token: 0x06000CAA RID: 3242 RVA: 0x00036DE2 File Offset: 0x00034FE2
			get;
			// Token: 0x06000CAB RID: 3243 RVA: 0x00036DEA File Offset: 0x00034FEA
			set;
		}

		// Token: 0x17000422 RID: 1058
		[HandlerParameter(true, "2", Min = "2", Max = "20", Step = "1")]
		public int MaMinPeriod
		{
			// Token: 0x06000CA8 RID: 3240 RVA: 0x00036DD1 File Offset: 0x00034FD1
			get;
			// Token: 0x06000CA9 RID: 3241 RVA: 0x00036DD9 File Offset: 0x00034FD9
			set;
		}

		// Token: 0x17000424 RID: 1060
		[HandlerParameter(true, "2", Min = "2", Max = "3", Step = "1")]
		public int Sharp
		{
			// Token: 0x06000CAC RID: 3244 RVA: 0x00036DF3 File Offset: 0x00034FF3
			get;
			// Token: 0x06000CAD RID: 3245 RVA: 0x00036DFB File Offset: 0x00034FFB
			set;
		}

		// Token: 0x17000426 RID: 1062
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool ShowNRTR
		{
			// Token: 0x06000CB0 RID: 3248 RVA: 0x00036E15 File Offset: 0x00035015
			get;
			// Token: 0x06000CB1 RID: 3249 RVA: 0x00036E1D File Offset: 0x0003501D
			set;
		}

		// Token: 0x17000425 RID: 1061
		[HandlerParameter(true, "3", Min = "2", Max = "10", Step = "1")]
		public int Smooth
		{
			// Token: 0x06000CAE RID: 3246 RVA: 0x00036E04 File Offset: 0x00035004
			get;
			// Token: 0x06000CAF RID: 3247 RVA: 0x00036E0C File Offset: 0x0003500C
			set;
		}
	}
}
