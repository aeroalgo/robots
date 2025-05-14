using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000138 RID: 312
	[HandlerCategory("vvRSI"), HandlerName("LaguerreRSI Variation")]
	public class LaguerreRSIvar : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x0600095A RID: 2394 RVA: 0x000273E6 File Offset: 0x000255E6
		public IList<double> Execute(IList<double> src)
		{
			return LaguerreRSIvar.GenLaguerreRSIvar(src, this.Gamma, this.RSIDataLevel, this.RSIPeriod, this.Smooth, this.Context);
		}

		// Token: 0x06000959 RID: 2393 RVA: 0x000271AC File Offset: 0x000253AC
		public static IList<double> GenLaguerreRSIvar(IList<double> price, double gamma, int _RSIDataLevel, int _RSIPeriod, bool smooth, IContext context)
		{
			int count = price.Count;
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double[] array4 = new double[count];
			double[] array5 = new double[count];
			double[] array6 = new double[count];
			_RSIDataLevel = Math.Max(Math.Min(_RSIDataLevel, 2), 0);
			for (int i = _RSIPeriod + 1; i < count; i++)
			{
				array3[i] = (1.0 - gamma) * price[i] + gamma * array3[i - 1];
				array4[i] = -gamma * array3[i] + array3[i - 1] + gamma * array4[i - 1];
				array5[i] = -gamma * array4[i] + array4[i - 1] + gamma * array5[i - 1];
				array6[i] = -gamma * array5[i] + array5[i - 1] + gamma * array6[i - 1];
				double num = 0.0;
				double num2 = 0.0;
				for (int j = 0; j < _RSIPeriod; j++)
				{
					double num3 = 0.0;
					switch (_RSIDataLevel)
					{
					case 0:
						num3 = array3[i - j] - array4[i - j];
						break;
					case 1:
						num3 = array4[i - j] - array5[i - j];
						break;
					case 2:
						num3 = array5[i - j] - array6[i - j];
						break;
					}
					if (num3 > 0.0)
					{
						num += num3;
					}
					if (num3 < 0.0)
					{
						num2 -= num3;
					}
				}
				if (num + num2 != 0.0)
				{
					array2[i] = 0.5 * ((num - num2) / (num + num2) + 1.0);
				}
				else
				{
					array2[i] = 0.0;
				}
				if (smooth)
				{
					array[i] = (array2[i] + 2.0 * array2[i - 1] + array2[i - 2]) / 4.0;
				}
				else
				{
					array[i] = array2[i];
				}
				array[i] *= 100.0;
			}
			return array;
		}

		// Token: 0x17000307 RID: 775
		public IContext Context
		{
			// Token: 0x0600095B RID: 2395 RVA: 0x0002740C File Offset: 0x0002560C
			get;
			// Token: 0x0600095C RID: 2396 RVA: 0x00027414 File Offset: 0x00025614
			set;
		}

		// Token: 0x17000303 RID: 771
		[HandlerParameter(true, "0.6", Min = "0", Max = "1", Step = "0.1")]
		public double Gamma
		{
			// Token: 0x06000951 RID: 2385 RVA: 0x00027168 File Offset: 0x00025368
			get;
			// Token: 0x06000952 RID: 2386 RVA: 0x00027170 File Offset: 0x00025370
			set;
		}

		// Token: 0x17000304 RID: 772
		[HandlerParameter(true, "0", Min = "0", Max = "2", Step = "1")]
		public int RSIDataLevel
		{
			// Token: 0x06000953 RID: 2387 RVA: 0x00027179 File Offset: 0x00025379
			get;
			// Token: 0x06000954 RID: 2388 RVA: 0x00027181 File Offset: 0x00025381
			set;
		}

		// Token: 0x17000305 RID: 773
		[HandlerParameter(true, "8", Min = "1", Max = "20", Step = "1")]
		public int RSIPeriod
		{
			// Token: 0x06000955 RID: 2389 RVA: 0x0002718A File Offset: 0x0002538A
			get;
			// Token: 0x06000956 RID: 2390 RVA: 0x00027192 File Offset: 0x00025392
			set;
		}

		// Token: 0x17000306 RID: 774
		[HandlerParameter(true, "true", NotOptimized = true)]
		public bool Smooth
		{
			// Token: 0x06000957 RID: 2391 RVA: 0x0002719B File Offset: 0x0002539B
			get;
			// Token: 0x06000958 RID: 2392 RVA: 0x000271A3 File Offset: 0x000253A3
			set;
		}
	}
}
