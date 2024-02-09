using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000171 RID: 369
	[HandlerCategory("vvAverages")]
	public class BaseAMA
	{
		// Token: 0x06000BA8 RID: 2984 RVA: 0x00031C74 File Offset: 0x0002FE74
		protected IList<double> Execute(IList<double> Price, bool isMama)
		{
			int count = Price.Count;
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double[] array4 = new double[count];
			double[] array5 = new double[count];
			double[] array6 = new double[count];
			double[] array7 = new double[count];
			double[] array8 = new double[count];
			double[] array9 = new double[count];
			double[] array10 = new double[count];
			double[] array11 = new double[count];
			double[] array12 = new double[count];
			for (int i = 0; i < 6; i++)
			{
				array11[i] = (array12[i] = Price[i]);
			}
			for (int j = 6; j < count; j++)
			{
				array[j] = (4.0 * Price[j] + 3.0 * Price[j - 1] + 2.0 * Price[j - 2] + Price[j - 3]) / 10.0;
				array2[j] = (0.0962 * array[j] + 0.5769 * array[j - 2] - 0.5769 * array[j - 4] - 0.0962 * array[j - 6]) * (0.075 * array7[j - 1] + 0.54);
				array4[j] = (0.0962 * array2[j] + 0.5769 * array2[j - 2] - 0.5769 * array2[j - 4] - 0.0962 * array2[j - 6]) * (0.075 * array7[j - 1] + 0.54);
				array3[j] = array2[j - 3];
				double num = (0.0962 * array3[j] + 0.5769 * array3[j - 2] - 0.5769 * array3[j - 4] - 0.0962 * array3[j - 6]) * (0.075 * array7[j - 1] + 0.54);
				double num2 = (0.0962 * array4[j] + 0.5769 * array4[j - 2] - 0.5769 * array4[j - 4] - 0.0962 * array4[j - 6]) * (0.075 * array7[j - 1] + 0.54);
				array5[j] = array3[j] - num2;
				array6[j] = array4[j] + num;
				array5[j] = 0.2 * array5[j] + 0.8 * array5[j - 1];
				array6[j] = 0.2 * array6[j] + 0.8 * array6[j - 1];
				array9[j] = array5[j] * array5[j - 1] + array6[j] * array6[j - 1];
				array10[j] = array5[j] * array6[j - 1] - array6[j] * array5[j - 1];
				array9[j] = 0.2 * array9[j] + 0.8 * array9[j - 1];
				array10[j] = 0.2 * array10[j] + 0.8 * array10[j - 1];
				if (array10[j] != 0.0 && array9[j] != 0.0)
				{
					array7[j] = 6.2831853071795862 / Math.Atan2(array10[j], array9[j]);
				}
				if (array7[j] > 1.5 * array7[j - 1])
				{
					array7[j] = 1.5 * array7[j - 1];
				}
				if (array7[j] < 0.67 * array7[j - 1])
				{
					array7[j] = 0.67 * array7[j - 1];
				}
				if (array7[j] < 6.0)
				{
					array7[j] = 6.0;
				}
				if (array7[j] > 50.0)
				{
					array7[j] = 50.0;
				}
				array7[j] = 0.2 * array7[j] + 0.8 * array7[j - 1];
				if (array3[j] != 0.0)
				{
					array8[j] = Math.Atan2(array4[j], array3[j]) * 180.0 / 3.1415926535897931;
				}
				double num3 = array8[j - 1] - array8[j];
				if (num3 < 1.0)
				{
					num3 = 1.0;
				}
				double num4 = this.FastLimit / num3;
				if (num4 < this.SlowLimit)
				{
					num4 = this.SlowLimit;
				}
				array11[j] = num4 * Price[j] + (1.0 - num4) * array11[j - 1];
				array12[j] = 0.5 * num4 * array11[j] + (1.0 - 0.5 * num4) * array12[j - 1];
			}
			if (!isMama)
			{
				return array12;
			}
			return array11;
		}

		// Token: 0x170003D5 RID: 981
		[HandlerParameter(true, "0.5", Min = "0.1", Max = "1.0", Step = "0.1")]
		public double FastLimit
		{
			// Token: 0x06000BA4 RID: 2980 RVA: 0x00031C51 File Offset: 0x0002FE51
			get;
			// Token: 0x06000BA5 RID: 2981 RVA: 0x00031C59 File Offset: 0x0002FE59
			set;
		}

		// Token: 0x170003D6 RID: 982
		[HandlerParameter(true, "0.05", Min = "0.01", Max = "0.1", Step = "0.01")]
		public double SlowLimit
		{
			// Token: 0x06000BA6 RID: 2982 RVA: 0x00031C62 File Offset: 0x0002FE62
			get;
			// Token: 0x06000BA7 RID: 2983 RVA: 0x00031C6A File Offset: 0x0002FE6A
			set;
		}
	}
}
