using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200001D RID: 29
	[HandlerCategory("vvIndicators"), HandlerDecimals(2), HandlerName("LaguerreCCI")]
	public class LaguerreCCI : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060000FB RID: 251 RVA: 0x00005449 File Offset: 0x00003649
		public IList<double> Execute(IList<double> src)
		{
			return LaguerreCCI.GenLaguerreCCI(src, this.Gamma, this.CCIperiod, this.NormaPeriod);
		}

		// Token: 0x060000FA RID: 250 RVA: 0x0000513C File Offset: 0x0000333C
		public static IList<double> GenLaguerreCCI(IList<double> src, double _gamma, int _cciperiod, int normaperiod)
		{
			int count = src.Count;
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double[] array4 = new double[count];
			double[] array5 = new double[count];
			double[] array6 = new double[count];
			double[] array7 = new double[count];
			double num = 0.015 / (double)_cciperiod;
			for (int i = 1; i < src.Count; i++)
			{
				array4[i] = (1.0 - _gamma) * src[i] + _gamma * array4[i - 1];
				array5[i] = -_gamma * array4[i] + array4[i - 1] + _gamma * array5[i - 1];
				array6[i] = -_gamma * array5[i] + array5[i - 1] + _gamma * array6[i - 1];
				array7[i] = -_gamma * array6[i] + array6[i - 1] + _gamma * array7[i - 1];
				double num2 = 0.0;
				double num3 = 0.0;
				if (array4[i] >= array5[i])
				{
					num2 = array4[i] - array5[i];
				}
				else
				{
					num3 = array5[i] - array4[i];
				}
				if (array5[i] >= array6[i])
				{
					num2 = num2 + array5[i] - array6[i];
				}
				else
				{
					num3 = num3 + array6[i] - array5[i];
				}
				if (array6[i] >= array7[i])
				{
					num2 = num2 + array6[i] - array7[i];
				}
				else
				{
					num3 = num3 + array7[i] - array6[i];
				}
				array[i] = (array4[i] + 2.0 * array5[i] + 2.0 * array6[i] + array7[i]) / 6.0;
			}
			for (int j = _cciperiod + 1; j < count; j++)
			{
				double num4 = 0.0;
				for (int k = j - _cciperiod - 1; k <= j; k++)
				{
					num4 += Math.Abs(src[k] - array[j]);
				}
				num4 = num * num4;
				double num5 = src[j] - array[j];
				if (num4 == 0.0)
				{
					array2[j] = 0.0;
				}
				else
				{
					array2[j] = num5 / num4;
				}
			}
			if (normaperiod > 0)
			{
				IList<double> list = vvSeries.Highest(array2, normaperiod);
				IList<double> list2 = vvSeries.Lowest(array2, normaperiod);
				for (int l = 0; l < count; l++)
				{
					if (array2[l] > 0.0)
					{
						double num6 = list[l];
						if (num6 != 0.0)
						{
							array3[l] = array2[l] / num6 * 100.0;
						}
					}
					if (array2[l] < 0.0)
					{
						double num7 = list2[l];
						if (num7 != 0.0)
						{
							array3[l] = array2[l] / -num7 * 100.0;
						}
					}
				}
			}
			if (normaperiod <= 0)
			{
				return array2;
			}
			return array3;
		}

		// Token: 0x17000050 RID: 80
		[HandlerParameter(true, "14", Min = "4", Max = "50", Step = "1")]
		public int CCIperiod
		{
			// Token: 0x060000F6 RID: 246 RVA: 0x0000511A File Offset: 0x0000331A
			get;
			// Token: 0x060000F7 RID: 247 RVA: 0x00005122 File Offset: 0x00003322
			set;
		}

		// Token: 0x17000052 RID: 82
		public IContext Context
		{
			// Token: 0x060000FC RID: 252 RVA: 0x00005463 File Offset: 0x00003663
			get;
			// Token: 0x060000FD RID: 253 RVA: 0x0000546B File Offset: 0x0000366B
			set;
		}

		// Token: 0x1700004F RID: 79
		[HandlerParameter(true, "0.5", Min = "0", Max = "1", Step = "0.1")]
		public double Gamma
		{
			// Token: 0x060000F4 RID: 244 RVA: 0x00005109 File Offset: 0x00003309
			get;
			// Token: 0x060000F5 RID: 245 RVA: 0x00005111 File Offset: 0x00003311
			set;
		}

		// Token: 0x17000051 RID: 81
		[HandlerParameter(true, "0", Min = "4", Max = "60", Step = "1")]
		public int NormaPeriod
		{
			// Token: 0x060000F8 RID: 248 RVA: 0x0000512B File Offset: 0x0000332B
			get;
			// Token: 0x060000F9 RID: 249 RVA: 0x00005133 File Offset: 0x00003333
			set;
		}
	}
}
